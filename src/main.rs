use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, terminal};
use std::collections::LinkedList;
use std::io::stdout;
use std::time::Duration;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode");
        Output::clear_screen().expect("Error");
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
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
        Self::draw_food(food)
    }
}

struct Game {
    reader: Reader,
    output: Output,
    snake: LinkedList<(u16, u16)>,
    food: (u16, u16),
}

impl Game {
    fn new() -> Self {
        let mut snake = LinkedList::new();
        snake.push_back((2, 2));
        Self {
            reader: Reader,
            snake,
            food: (20, 10),
            output: Output::new(),
        }
    }

    fn process_keypress(&self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return Ok(false),
            _ => {}
        }
        Ok(true)
    }

    fn run(&self) -> crossterm::Result<bool> {
        self.output.refresh_screen(&self.snake, &self.food)?;
        self.process_keypress()
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let game = Game::new();
    while game.run()? {}
    Ok(())
}
