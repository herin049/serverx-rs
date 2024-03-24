#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AxisProperty {
    X,
    Y,
    Z,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FacingProperty {
    Down,
    East,
    North,
    South,
    Up,
    West,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InstrumentProperty {
    Banjo,
    Basedrum,
    Bass,
    Bell,
    Bit,
    Chime,
    CowBell,
    Creeper,
    CustomHead,
    Didgeridoo,
    Dragon,
    Flute,
    Guitar,
    Harp,
    Hat,
    IronXylophone,
    Piglin,
    Pling,
    Skeleton,
    Snare,
    WitherSkeleton,
    Xylophone,
    Zombie,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PartProperty {
    Foot,
    Head,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShapeProperty {
    AscendingEast,
    AscendingNorth,
    AscendingSouth,
    AscendingWest,
    EastWest,
    InnerLeft,
    InnerRight,
    NorthEast,
    NorthSouth,
    NorthWest,
    OuterLeft,
    OuterRight,
    SouthEast,
    SouthWest,
    Straight,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HalfProperty {
    Bottom,
    Lower,
    Top,
    Upper,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KindProperty {
    Bottom,
    Double,
    Left,
    Normal,
    Right,
    Single,
    Sticky,
    Top,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EastProperty {
    Low,
    None,
    Side,
    Tall,
    Up,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NorthProperty {
    Low,
    None,
    Side,
    Tall,
    Up,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SouthProperty {
    Low,
    None,
    Side,
    Tall,
    Up,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WestProperty {
    Low,
    None,
    Side,
    Tall,
    Up,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HingeProperty {
    Left,
    Right,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceProperty {
    Ceiling,
    Floor,
    Wall,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ModeProperty {
    Compare,
    Corner,
    Data,
    Load,
    Save,
    Subtract,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LeavesProperty {
    Large,
    None,
    Small,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AttachmentProperty {
    Ceiling,
    DoubleWall,
    Floor,
    SingleWall,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrientationProperty {
    DownEast,
    DownNorth,
    DownSouth,
    DownWest,
    EastUp,
    NorthUp,
    SouthUp,
    UpEast,
    UpNorth,
    UpSouth,
    UpWest,
    WestUp,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SculkSensorPhaseProperty {
    Active,
    Cooldown,
    Inactive,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ThicknessProperty {
    Base,
    Frustum,
    Middle,
    Tip,
    TipMerge,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VerticalDirectionProperty {
    Down,
    Up,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TiltProperty {
    Full,
    None,
    Partial,
    Unstable,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TrialSpawnerStateProperty {
    Active,
    Cooldown,
    EjectingReward,
    Inactive,
    WaitingForPlayers,
    WaitingForRewardEjection,
}
