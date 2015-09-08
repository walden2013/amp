extern crate scribe;
extern crate luthor;

use models::application::Application;
use scribe::buffer::Position;
use self::luthor::token::Category;
use super::{application, buffer};

pub fn move_up(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_up(),
        None => (),
    }
}

pub fn move_down(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_down(),
        None => (),
    }
}

pub fn move_left(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_left(),
        None => (),
    }
}

pub fn move_right(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_right(),
        None => (),
    }
}

pub fn move_to_start_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_to_start_of_line(),
        None => (),
    }
}

pub fn move_to_first_word_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            // Get the current line.
            match buffer.data().lines().nth(buffer.cursor.line) {
                Some(line) => {
                    // Find the offset of the first non-whitespace character.
                    for (offset, character) in line.chars().enumerate() {
                        if !character.is_whitespace() {
                            // Move the cursor to this position.
                            let new_cursor_position = scribe::buffer::Position{
                                line: buffer.cursor.line,
                                offset: offset,
                            };
                            buffer.cursor.move_to(new_cursor_position);

                            // Stop enumerating; we've done the job.
                            return
                        }
                    }
                },
                None => ()
            }
        },
        None => ()
    }
}

pub fn move_to_end_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_to_end_of_line(),
        None => (),
    }
}

pub fn insert_at_end_of_line(app: &mut Application) {
    move_to_end_of_line(app);
    application::switch_to_insert_mode(app);
}

pub fn insert_at_first_word_of_line(app: &mut Application) {
    move_to_first_word_of_line(app);
    application::switch_to_insert_mode(app);
}

pub fn insert_with_newline(app: &mut Application) {
    move_to_end_of_line(app);
    buffer::start_command_group(app);
    buffer::insert_newline(app);
    application::switch_to_insert_mode(app);
}

pub fn move_to_start_of_previous_token(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            let tokens = buffer.tokens();
            let mut line = 0;
            let mut offset = 0;
            let mut closest_position = Position{ line: 0, offset: 0 };
            let mut next_position = closest_position;
            for token in tokens.iter() {
                closest_position = match token.category {
                    Category::Whitespace => closest_position,
                    _ => next_position,
                };

                // Calculate the position of the next token.
                match token.lexeme.lines().count() {
                    1 => {
                        // There's only one line in this token, so
                        // only advance the offset by its size.
                        offset += token.lexeme.len()
                    },
                    n => {
                        // There are multiple lines, so advance the
                        // line count and set the offset to the last
                        // line's length
                        line += n-1;
                        offset = token.lexeme.lines().last().unwrap().len();
                    },
                };

                next_position = Position{ line: line, offset: offset };

                if next_position >= *buffer.cursor {
                    break;
                }
            };

            buffer.cursor.move_to(closest_position);
        },
        None => (),
    }
}

pub fn move_to_start_of_next_token(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            let tokens = buffer.tokens();
            let mut line = 0;
            let mut offset = 0;
            let mut next_position = Position{ line: 0, offset: 0 };
            for token in tokens.iter() {
                if next_position > *buffer.cursor {
                    match token.category {
                        Category::Whitespace => (),
                        _ => {
                            buffer.cursor.move_to(next_position);
                            break
                        }
                    };
                }

                // Calculate the position of the next token.
                match token.lexeme.lines().count() {
                    1 => {
                        // There's only one line in this token, so
                        // only advance the offset by its size.
                        offset += token.lexeme.len()
                    },
                    n => {
                        // There are multiple lines, so advance the
                        // line count and set the offset to the last
                        // line's length
                        line += n-1;
                        offset = token.lexeme.lines().last().unwrap().len();
                    },
                };

                next_position = Position{ line: line, offset: offset };
            };

        },
        None => (),
    }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    #[test]
    fn move_to_first_word_of_line_works() {
        let mut app = ::models::application::new();
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp");
        let position = scribe::buffer::Position{ line: 0, offset: 7};
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::move_to_first_word_of_line(&mut app);

        // Ensure that the cursor is moved to the start of the first word.
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, 0);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset, 4);
    }

    #[test]
    fn move_to_start_of_previous_token_works() {
        let mut app = ::models::application::new();
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("amp editor");
        let position = scribe::buffer::Position{ line: 0, offset: 7};
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::move_to_start_of_previous_token(&mut app);

        // Ensure that the cursor is moved to the start of the previous word.
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, 0);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset, 4);
    }

    #[test]
    fn move_to_start_of_previous_token_skips_whitespace() {
        let mut app = ::models::application::new();
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("amp editor");
        let position = scribe::buffer::Position{ line: 0, offset: 7};
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::move_to_start_of_previous_token(&mut app);
        super::move_to_start_of_previous_token(&mut app);

        // Ensure that the cursor is moved to the start of the previous word.
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, 0);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset, 0);
    }

    #[test]
    fn move_to_start_of_next_token_works() {
        let mut app = ::models::application::new();
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("amp editor");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::move_to_start_of_next_token(&mut app);

        // Ensure that the cursor is moved to the start of the previous word.
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, 0);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset, 4);
    }
}