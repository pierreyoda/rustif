use std::fs::File;
use std::path::Path;

use crate::errors::IFtResult;
use rustifzm::ZMachine;

/// The Interactive Fiction Terminal Client is the frontend interface
/// used to play a story file by managing user input and game output.
pub struct IFTerminalClient {
    vm: ZMachine,
}

impl IFTerminalClient {
    pub fn with_story_file(story_path: &Path) -> IFtResult<Self> {
        let mut story_file = File::open(story_path)?;
        let vm = ZMachine::from_story_reader(&mut story_file)?;
        Ok(IFTerminalClient { vm })
    }

    pub fn run(&mut self) -> IFtResult<()> {
        for _ in 0..10 {
            self.vm.step()?;
        }
        Ok(())
    }
}
