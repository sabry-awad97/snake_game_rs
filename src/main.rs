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

struct Output;

impl Output {
    fn new() -> Self {
        Self
    }

    fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn refresh_screen(&self, snake: &Snake, food: &Food) -> crossterm::Result<()> {
        Self::clear_screen()?;
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

    fn draw(&self) -> crossterm::Result<()> {
        for &(x, y) in &self.segments {
            execute!(stdout(), cursor::MoveTo(x, y), crossterm::style::Print("█"))?;
        }
        Ok(())
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

        Self {
            output: Output::new(),
            snake,
            food,
        }
    }

    fn update_snake(&mut self) -> bool {
        let (x, y) = self.snake.segments.front().unwrap().clone();
        let new_head = match self.snake.direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };

        // Check for collision with walls
        if new_head.0 == 0 || new_head.0 >= WIDTH - 1 || new_head.1 == 0 || new_head.1 >= HEIGHT - 1
        {
            return false;
        }

        // Check for collision with self
        if self.snake.segments.contains(&new_head) {
            return false;
        }

        self.snake.segments.push_front(new_head);

        // Check for collision with food
        if new_head == self.food.position() {
            self.food.respawn()
        } else {
            self.snake.segments.pop_back();
        }

        true
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.snake.direction = Direction::Up,
            KeyCode::Down => self.snake.direction = Direction::Down,
            KeyCode::Left => self.snake.direction = Direction::Left,
            KeyCode::Right => self.snake.direction = Direction::Right,
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
    let mut game = Game::new();

    let mut last_update = Instant::now();
    let update_rate = Duration::from_millis(100);

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
