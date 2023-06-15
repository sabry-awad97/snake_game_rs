use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, terminal};
use std::collections::LinkedList;
use std::io::stdout;
use std::time::{Duration, Instant};

const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;

#[derive(Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode");
        Output::clear_screen().expect("Error");
    }
}

struct Output {
    border_printed: bool,
}

impl Output {
    fn new() -> Self {
        Self {
            border_printed: false,
        }
    }

    fn clear_inner(_snake: &Snake, food: &Food) -> crossterm::Result<()> {
        for y in 1..HEIGHT - 1 {
            for x in 1..WIDTH - 1 {
                let is_food = (x, y) == food.position();
                if !is_food {
                    execute!(stdout(), cursor::MoveTo(x, y), crossterm::style::Print(" "))?;
                }
            }
        }
        Ok(())
    }

    fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), cursor::MoveTo(0, 0))?;
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        Ok(())
    }

    fn print_border(&mut self) -> crossterm::Result<()> {
        if !self.border_printed {
            // Top border
            for x in 0..WIDTH {
                execute!(stdout(), cursor::MoveTo(x, 0), crossterm::style::Print("─"))?;
            }
            execute!(stdout(), cursor::MoveTo(0, 0), crossterm::style::Print("┌"))?;
            execute!(
                stdout(),
                cursor::MoveTo(WIDTH - 1, 0),
                crossterm::style::Print("┐")
            )?;

            // Bottom border
            for x in 0..WIDTH {
                execute!(
                    stdout(),
                    cursor::MoveTo(x, HEIGHT - 1),
                    crossterm::style::Print("─")
                )?;
            }
            execute!(
                stdout(),
                cursor::MoveTo(0, HEIGHT - 1),
                crossterm::style::Print("└")
            )?;
            execute!(
                stdout(),
                cursor::MoveTo(WIDTH - 1, HEIGHT - 1),
                crossterm::style::Print("┘")
            )?;

            // Left border
            for y in 0..HEIGHT {
                execute!(stdout(), cursor::MoveTo(0, y), crossterm::style::Print("│"))?;
            }
            execute!(stdout(), cursor::MoveTo(0, 0))?;

            // Right border
            for y in 0..HEIGHT {
                execute!(
                    stdout(),
                    cursor::MoveTo(WIDTH - 1, y),
                    crossterm::style::Print("│")
                )?;
            }
            execute!(stdout(), cursor::MoveTo(WIDTH - 1, 0))?;
        }
        self.border_printed = true;

        Ok(())
    }

    fn refresh_screen(&mut self, snake: &Snake, food: &Food) -> crossterm::Result<()> {
        execute!(stdout(), cursor::Hide)?;
        Self::clear_inner(snake, food)?;
        self.print_border()?;
        snake.draw()?;
        food.draw()?;
        Ok(())
    }
}

struct Snake {
    segments: LinkedList<(u16, u16)>,
    direction: Direction,
}

impl Snake {
    fn new(segments: LinkedList<(u16, u16)>, direction: Direction) -> Self {
        Self {
            segments,
            direction,
        }
    }

    fn change_direction(&mut self, direction: Direction) {
        // Prevent the snake from reversing its direction
        match (self.direction.clone(), direction.clone()) {
            (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up)
            | (Direction::Left, Direction::Right)
            | (Direction::Right, Direction::Left) => {}
            _ => self.direction = direction,
        }
    }

    fn check_self_collision(&self, new_head: (u16, u16)) -> bool {
        self.segments.contains(&new_head)
    }

    fn check_wall_collision(&self, new_head: (u16, u16)) -> bool {
        new_head.0 == 0 || new_head.0 >= WIDTH - 1 || new_head.1 == 0 || new_head.1 >= HEIGHT - 1
    }

    fn check_food_collision(&mut self, new_head: (u16, u16), food: &mut Food) -> bool {
        new_head == food.position()
    }

    fn head(&mut self) -> (u16, u16) {
        self.segments.front().unwrap().clone()
    }

    fn set_head(&mut self, new_head: (u16, u16)) {
        self.segments.push_front(new_head)
    }

    fn remove_last_segment(&mut self) {
        self.segments.pop_back();
    }

    fn draw(&self) -> crossterm::Result<()> {
        for &(x, y) in &self.segments {
            execute!(stdout(), cursor::MoveTo(x, y), crossterm::style::Print("█"))?;
        }
        Ok(())
    }

    fn move_in_direction(&mut self) -> (u16, u16) {
        let (x, y) = self.head();
        match self.direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        }
    }
}

struct Food {
    x: u16,
    y: u16,
}

struct Game {
    output: Output,
    snake: Snake,
    food: Food,
}

impl Food {
    fn new() -> Self {
        Self { x: 20, y: 10 }
    }

    fn position(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    fn respawn(&mut self) {
        self.x = 1 + rand::random::<u16>() % (WIDTH - 2);
        self.y = 1 + rand::random::<u16>() % (HEIGHT - 2);
    }

    fn draw(&self) -> crossterm::Result<()> {
        execute!(
            stdout(),
            cursor::MoveTo(self.x, self.y),
            crossterm::style::Print("@")
        )?;
        Ok(())
    }
}

impl Game {
    fn new() -> Self {
        let mut segments = LinkedList::new();
        segments.push_back((2, 2));
        let snake = Snake::new(segments, Direction::Right);

        let food = Food::new();
        let output = Output::new();

        Self {
            snake,
            food,
            output,
        }
    }

    fn update_snake(&mut self) -> bool {
        let new_head = self.snake.move_in_direction();

        if self.snake.check_self_collision(new_head) || self.snake.check_wall_collision(new_head) {
            return false;
        }

        self.snake.set_head(new_head);

        // Check for collision with food
        if self.snake.check_food_collision(new_head, &mut self.food) {
            self.food.respawn()
        } else {
            self.snake.remove_last_segment();
        }

        true
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.snake.change_direction(Direction::Up),
            KeyCode::Down => self.snake.change_direction(Direction::Down),
            KeyCode::Left => self.snake.change_direction(Direction::Left),
            KeyCode::Right => self.snake.change_direction(Direction::Right),
            _ => {}
        }
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen(&self.snake, &self.food)?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(event) = event::read()? {
                self.handle_key_event(event);
            }
        }

        if !self.update_snake() {
            return Ok(false);
        }

        Ok(true)
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    Output::clear_screen()?;

    let mut game = Game::new();

    let mut last_update = Instant::now();
    let update_rate = Duration::from_millis(10);

    loop {
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(event) = event::read()? {
                if event.code == KeyCode::Char('q')
                    && event.modifiers == event::KeyModifiers::CONTROL
                {
                    break;
                }
            }
        }

        if Instant::now() - last_update >= update_rate {
            if !game.run()? {
                break;
            }
            last_update = Instant::now();
        }
    }

    Ok(())
}
