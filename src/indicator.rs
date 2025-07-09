/// Singals of the file
pub enum IndicateSignal {
    /// Failed download
    Fail(String),
    /// Change of state
    State(String),
    /// Success download
    Success(),
    /// Start download
    Start(),
}

/// Trait for creation of indicators
pub trait IndicatorFactory: Default {
    /// Creates a new indicator for file with size
    fn create_task(&self, name: &str, size: u64) -> impl Indicator;
}
/// Trait for indicator in one single file
pub trait Indicator {
    /// Callback for change of progress bar
    fn effect(&mut self, position: u64);
    /// Callback for signals
    fn signal(&mut self, signal: IndicateSignal);
}

/// Silent default indicator, don't print any thing
#[derive(Default)]
pub struct Silent;
impl IndicatorFactory for Silent {
    fn create_task(&self, name: &str, size: u64) -> impl Indicator {
        SilentChild
    }
}
// Child of silent indicator
pub struct SilentChild;
impl Indicator for SilentChild {
    fn signal(&mut self, signal: IndicateSignal) {}
    fn effect(&mut self, position: u64) {}
}

#[cfg(feature = "indicatif_indicator")]
pub mod indicatif {
    use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
    /// Indicatif indicator implementation
    pub struct Indicatif {
        style: ProgressStyle,
        multiprogress: MultiProgress,
    }
    impl super::IndicatorFactory for Indicatif {
        fn create_task(&self, name: &str, size: u64) -> IndicatifChild {
            let bar = ProgressBar::new(size).with_style(self.style.clone());
            bar.set_draw_target(ProgressDrawTarget::hidden());
            let bar = self.multiprogress.add(bar);
            bar.set_message(name.to_string());
            IndicatifChild { bar }
        }
    }
    impl Default for Indicatif {
        fn default() -> Self {
            Self {
                style: ProgressStyle::default_bar(),
                multiprogress: MultiProgress::new(),
            }
        }
    }
    impl Indicatif {
        pub fn new(style: ProgressStyle) -> Self {
            Self {
                style,
                multiprogress: MultiProgress::new(),
            }
        }
    }

    pub struct IndicatifChild {
        bar: ProgressBar,
    }
    impl super::Indicator for IndicatifChild {
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
                super::IndicateSignal::Start() => {
                    self.bar.set_draw_target(ProgressDrawTarget::stdout());
                }
            }
        }
    }
}
