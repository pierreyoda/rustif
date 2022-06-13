use std::fs::File;

use rustifzm::ZMachine;

const CPU_STEPS_LIMIT: usize = 10_000_000; // TODO: detect test ending?

fn setup(test_story_path: &str) -> ZMachine {
    let mut test_story_file = File::open(test_story_path).expect("should open the test file");
    ZMachine::from_story_reader(&mut test_story_file).expect("should init harness ZMachine")
}

macro_rules! run_story_tests_files {
    ($ ( $name: ident : $filename: expr, )* ) => {
    $(
        #[test]
        fn $name() {
            let story_path = format!("./tests/{}", $filename);
            let mut zmachine = setup(&story_path);
            for _ in 0..CPU_STEPS_LIMIT {
                zmachine.step().expect("should step the instruction properly");
            }
        }
    )*
    }
}

run_story_tests_files! {
    test_czech: "czech_0_8/czech.z5",
    test_praxix: "praxix.z5",
}
