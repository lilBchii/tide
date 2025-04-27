use editing::Editing;
use welcome::Welcome;

pub mod component;
pub mod editing;
pub mod welcome;

pub enum Screen {
    Editing(Editing),
    Welcome(Welcome),
}
