#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Group {
    UseCase(UseCase),
    MathematicalBasis(MathematicalBasis),
    DataInputType(DataInputType),
    SignalType(SignalType),
    OutputFormat(OutputFormat),
    TimeframeFocus(TimeframeFocus),
    ComplexityLevel(ComplexityLevel),
    MarketSuitability(MarketSuitability),
    TradingStrategySuitability(TradingStrategySuitability),
    SmoothingTechnique(SmoothingTechnique),
    CalculationMethodology(CalculationMethodology),
    SignalInterpretation(SignalInterpretation),
}

/// Classifies indicators based on their primary function or the problem they solve
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum UseCase {
    TrendIdentification,
    MomentumDetection,
    VolatilityMeasurement,
    VolumeConfirmation,
    ReversalDetection,
    CycleAnalysis,
    PatternRecognition,
    SupportResistanceLevels,
    MarketStrengthMeasurement,
    PriceTransformation,
}

// TODO: discuss that group about mathematical basis. Seems like usecases are mixed here
/// Groups indicators based on the mathematical techniques or formulas they employ
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MathematicalBasis {
    /// Calculating averages over time.
    MovingAverage,
    /// Mathematical functions that oscillate within a range.
    Oscillator,
    /// Statistical measures of price variability or dispersion.
    VolatilityMeasure,
    /// Statistical methods for modeling relationships between variables.
    RegressionAnalysis,
    /// Mathematical transformations like normalization or standardization.
    StatisticalTransformation,
    /// Decomposing signals into their frequency components.
    FourierTransform,
    /// Analyzing the instantaneous attributes of signals.
    HilbertTransform,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DataInputType {
    PriceBased,
    VolumeBased,
    PriceVolumeCombined,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SignalType {
    /// Predict future price movements.
    Leading,
    /// Confirm existing trends after they have begun.
    Lagging,
    /// Move simultaneously with the market.
    Coincident,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum OutputFormat {
    SingleLine,
    MultiLine,
    Band,
    Histogram,
    Percentage,
    Absolute,
    Directional
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TimeframeFocus {
    /// intraday to few days
    Short,
    /// weeks to months
    Medium,
    /// months to years
    Long
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ComplexityLevel {
    Basic,
    Intermediate,
    Advanced
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MarketSuitability {
    /// Works well in markets with clear trends
    Trending,
    /// Effective in sideways or consolidating markets
    RangeBound,
    /// Designed to handle high volatility
    Volatile,
    /// Best in low volatility environments
    Stable,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TradingStrategySuitability {
    /// short-term trading strategies, ms.
    Scalping,
    /// intraday trading strategies
    Intraday,
    /// Holding positions for days to weeks
    Swing,
    /// Long-term trading strategies
    Positional,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SmoothingTechnique {
    SimpleAverage,
    Exponential,
    WeightedMovingAverage,
    Adaptive
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum CalculationMethodology {
    /// Accumulates data over time (e.g., Accumulation/Distribution Line).
    Cumulative,
    /// Uses differences between values (e.g., MACD).
    Differential,
    /// Computes ratios (e.g., Relative Strength).
    Ratio,
    /// Employs statistical measures (e.g., Standard Deviation).
    Statistical,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SignalInterpretation {
    /// overbought / oversold
    PeakThroughLevels,
    /// Signals generated when lines cross.
    Crossovers,
    /// Discrepancies between indicator and price movement.
    Divergence,
    /// Signals based on surpassing certain levels.
    ThresholdLevels,
    /// Specific patterns in the indicator output
    Patterns,
}