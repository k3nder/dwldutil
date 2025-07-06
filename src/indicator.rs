pub enum IndicateSignal {
    Fail(String),
    State(String),
    Success(),
}

pub trait IndicatorFactory: Default {
    fn create_task(&self, name: &str, size: u64) -> impl Indicator;
}
pub trait Indicator {
    fn effect(&mut self, position: u64);
    fn signal(&mut self, signal: IndicateSignal);
}

#[cfg(feature = "indicatif_progress")]
pub mod indicatif {
    use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
    pub struct IndicatifFactory {
        style: ProgressStyle,
        multiprogress: MultiProgress,
    }
    impl super::IndicatorFactory for IndicatifFactory {
        fn create_task(&self, name: &str, size: u64) -> Indicatif {
            let bar = ProgressBar::new(size).with_style(self.style.clone());
            let bar = self.multiprogress.add(bar);

            Indicatif { bar }
        }
    }
    impl Default for IndicatifFactory {
        fn default() -> Self {
            Self {
                style: ProgressStyle::default_bar(),
                multiprogress: MultiProgress::new(),
            }
        }
    }

    pub struct Indicatif {
        bar: ProgressBar,
    }
    impl super::Indicator for Indicatif {
        fn effect(&mut self, position: u64) {
            self.bar.set_position(position);
        }
        fn signal(&mut self, signal: super::IndicateSignal) {
            match signal {
                super::IndicateSignal::Fail(f) => {
                    self.bar.finish_with_message(format!("Error -- {}", f));
                }
                super::IndicateSignal::State(s) => {
                    self.bar.set_message(s);
                }
                super::IndicateSignal::Success() => {
                    self.bar.finish_with_message(format!("Done!"));
                }
            }
        }
    }
}
