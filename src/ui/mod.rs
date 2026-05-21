use std::io::{self, Stdout};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    widgets::Paragraph,
};

use crate::domain::{Position, ResourceType, RobotKind, WorldSnapshot};

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn start_terminal() -> io::Result<AppTerminal> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn stop_terminal(terminal: &mut AppTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn render_world<B: Backend>(
    terminal: &mut Terminal<B>,
    snapshot: &WorldSnapshot,
) -> io::Result<()> {
    terminal.draw(|frame| draw_world(frame, snapshot))?;
    Ok(())
}

pub fn draw_world(frame: &mut Frame, snapshot: &WorldSnapshot) {
    let text = display_lines(snapshot).join("\n");
    let paragraph = Paragraph::new(text);

    frame.render_widget(paragraph, frame.area());
}

pub fn display_lines(snapshot: &WorldSnapshot) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!("Tick: {}", snapshot.tick));
    lines.push(format!(
        "Energie: {} | Cristaux: {}",
        snapshot.collected_energy, snapshot.collected_crystals
    ));
    lines.push(String::new());

    for line in map_lines(snapshot) {
        lines.push(line);
    }

    lines
}

pub fn map_lines(snapshot: &WorldSnapshot) -> Vec<String> {
    let (width, height) = map_size(snapshot);
    let mut cells = vec![vec!['.'; width]; height];

    place(&mut cells, snapshot.base_position, 'B');

    for pos in &snapshot.obstacles {
        place(&mut cells, *pos, '#');
    }

    for resource in &snapshot.resources {
        let value = match resource.resource_type {
            ResourceType::Energy => 'E',
            ResourceType::Crystal => 'C',
        };

        place(&mut cells, resource.position, value);
    }

    for robot in &snapshot.robots {
        let value = match robot.kind {
            RobotKind::Scout => 'S',
            RobotKind::Collector => 'R',
        };

        place(&mut cells, robot.position, value);
    }

    let mut lines = Vec::new();

    for row in cells {
        lines.push(row.into_iter().collect());
    }

    lines
}

fn map_size(snapshot: &WorldSnapshot) -> (usize, usize) {
    let mut width = snapshot.base_position.x + 1;
    let mut height = snapshot.base_position.y + 1;

    for pos in &snapshot.obstacles {
        grow_size(*pos, &mut width, &mut height);
    }

    for resource in &snapshot.resources {
        grow_size(resource.position, &mut width, &mut height);
    }

    for robot in &snapshot.robots {
        grow_size(robot.position, &mut width, &mut height);
    }

    if width < 1 {
        width = 1;
    }

    if height < 1 {
        height = 1;
    }

    (width as usize, height as usize)
}

fn grow_size(pos: Position, width: &mut i32, height: &mut i32) {
    if pos.x >= *width {
        *width = pos.x + 1;
    }

    if pos.y >= *height {
        *height = pos.y + 1;
    }
}

fn place(cells: &mut [Vec<char>], pos: Position, value: char) {
    if pos.x < 0 || pos.y < 0 {
        return;
    }

    let y = pos.y as usize;
    let x = pos.x as usize;

    if y >= cells.len() || x >= cells[y].len() {
        return;
    }

    cells[y][x] = value;
}

pub fn register() {}
