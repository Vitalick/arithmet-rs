use std::fmt::Display;

pub enum Grade {
    Five = 5,
    Four = 4,
    Three = 3,
    Two = 2,
    One = 1,
}

impl Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::Five => write!(f, "Отлично"),
            Grade::Four => write!(f, "Хорошо"),
            Grade::Three => write!(f, "Удовлетворительно"),
            Grade::Two => write!(f, "Неудовлетворительно"),
            Grade::One => write!(f, "Плохо"),
        }
    }
}

impl Grade {
    pub fn from_quantity(correct_answers: usize, total_answers: usize) -> Grade {
        if total_answers == 0 {
            panic!("Общее количество ответов не может быть нулевым");
        }
        if correct_answers > total_answers {
            panic!("Количество правильных ответов не может быть больше общего количества ответов");
        }
        if correct_answers == total_answers {
            return Grade::from_percent(100.0);
        }
        let percent = (correct_answers as f32) * 100.0 / (total_answers as f32);
        Grade::from_percent(percent)
    }
    pub fn from_percent(percent: f32) -> Grade {
        if percent > 100.0 {
            panic!("{}% > 100%", percent);
        }
        if percent >= 98.0 {
            return Grade::Five;
        }
        if percent >= 75.0 {
            return Grade::Four;
        }
        if percent >= 50.0 {
            return Grade::Three;
        }
        if percent >= 20.0 {
            return Grade::Two;
        }
        Grade::One
    }

    pub fn full_banner(&self) -> [&str; 6] {
        const BASE_BANNER: [&str; 6] = [
            "███   ██  █ █ █  ██     ██  █  █  ████ █  █ █  █  ██",
            "█  █ █  █ █ █ █ █  █   █  █ █  █  █    █  █ █ █  █  █",
            "███  ████ █ █ █ ████   █  █ █  █  ███  ████ ██   ████",
            "█  █ █  █ █ █ █ █  █   █  █ █  █  █    █  █ █ █  █  █",
            "███  █  █ █████ █  █    ██  █████ ████ █  █ █  █ █  █",
            "                                █                    "
        ];
        let grade_banner = self.banner();

        const SPACE_BANNER: [&str; 6] = [
            "   ",
            "   ",
            "   ",
            "   ",
            "   ",
            "   "
        ];

        let final_banner = grade_banner.iter().zip(SPACE_BANNER.iter()).map(|(a, b)| a.to_string() + b).collect::<Vec<_>>();

    }

    pub fn banner(&self) -> [&str; 6] {
        match self {
            Grade::Five =>
                [
                "████",
                "█   ",
                "████",
                "   █",
                "████",
                "    "
                ],
            Grade::Four => [
                "█  █",
                "█  █",
                "████",
                "   █",
                "   █",
                "    "
            ],
            Grade::Three => [
                " ██ ",
                "█  █",
                "  █ ",
                "█  █",
                " ██ ",
                "    "
            ],
            Grade::Two => [
                " ██",
                "█  █",
                "  █",
                " █",
                "████",
                "    "
            ],
            Grade::One => [
                "   █",
                "  ██",
                "   █",
                "   █",
                "   █",
                "    "
            ]
        }

    }
}
