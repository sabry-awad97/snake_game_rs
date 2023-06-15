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

    fn draw_snake(snake: &LinkedList<(u16, u16)>) -> crossterm::Result<()> {
        for &(x, y) in snake {
            execute!(stdout(), cursor::MoveTo(x, y), crossterm::style::Print("█"))?;
        }
        Ok(())
    }

    fn draw_food(food: &(u16, u16)) -> crossterm::Result<()> {
        execute!(
            stdout(),
            cursor::MoveTo(food.0, food.1),
            crossterm::style::Print("@")
        )?;
        Ok(())
    }

    fn refresh_screen(
        &self,
        snake: &LinkedList<(u16, u16)>,
        food: &(u16, u16),
    ) -> crossterm::Result<()> {
        Self::clear_screen()?;
        Self::draw_snake(snake)?;
        Self::draw_food(food)?;
        Ok(())
    }
}

struct Game {
    output: Output,
    snake: LinkedList<(u16, u16)>,
    food: (u16, u16),
    direction: Direction,
}

impl Game {
    fn new() -> Self {
        let mut snake = LinkedList::new();
        snake.push_back((2, 2));

        Self {
            output: Output::new(),
            snake,
            food: (20, 10),
            direction: Direction::Right,
        }
    }

    fn update_snake(&mut self) -> bool {
        let (x, y) = self.snake.front().unwrap().clone();
        let new_head = match self.direction {
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
        if self.snake.contains(&new_head) {
            return false;
        }

        self.snake.push_front(new_head);

        // Check for collision with food
        if new_head == self.food {
            self.food = (
                1 + rand::random::<u16>() % (WIDTH - 2),
                1 + rand::random::<u16>() % (HEIGHT - 2),
            );
        } else {
            self.snake.pop_back();
        }

        true
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen(&self.snake, &self.food)?;

        if !self.update_snake() {
            return Ok(false);
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(event) = event::read()? {
                match event {
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } => self.direction = Direction::Up,
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => self.direction = Direction::Down,
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => self.direction = Direction::Left,
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => self.direction = Direction::Right,
                    _ => {}
                };
            }
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
