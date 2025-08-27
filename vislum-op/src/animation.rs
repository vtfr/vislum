use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::TaggedValue;

/// The interpolation mode for an animation.
///
/// Interpolates between two values.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interpolation {
    /// Interpolates between two values using a linear interpolation.
    Linear,
}

pub trait Interpolate {
    fn interpolate(&self, a: Self, b: Self, control: f32, interpolation: Interpolation) -> Self
    where
        Self: Sized;
}

impl Interpolate for f32 {
    #[inline(always)]
    fn interpolate(&self, a: Self, b: Self, control: f32, interpolation: Interpolation) -> Self {
        match interpolation {
            Interpolation::Linear => a + (b - a) * control,
        }
    }
}

/// A keyframe in an animation.
pub struct Keyframe<T> {
    pub time: u32,
    pub value: T,
    pub interpolation: Interpolation,
}

pub struct Animation<T> {
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T> Default for Animation<T> {
    fn default() -> Self {
        Self { keyframes: vec![] }
    }
}

#[derive(Error, Debug)]
pub enum AnimationError {
    #[error("The keyframe time is not unique: {0}")]
    NonUniqueKeyframeTime(u32),
}

impl<T> Animation<T>
where
    T: Interpolate,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a keyframe to the animation.
    pub fn add_keyframe(&mut self, time: u32, value: T, interpolation: Interpolation) -> Result<(), AnimationError> {
        let insert_index = match self.find_keyframe(time) {
            Ok(_) => return Err(AnimationError::NonUniqueKeyframeTime(time)),
            Err(index) => index,
        };

        self.keyframes.insert(insert_index, Keyframe { time, value, interpolation });

        Ok(())
    }

    pub fn animate(&self, time: u32) -> T {
        // let (keyframe, index) = self.find_keyframe(time).unwrap(); keyframe.value
        todo!()
    }

    /// Finds the keyframe at the given time.
    ///
    /// Returns the index of the keyframe if it exists, otherwise the index
    /// where it would be inserted.
    fn find_keyframe(&self, time: u32) -> Result<(&Keyframe<T>, usize), usize> {
        let index = self.keyframes.binary_search_by_key(&time, |k| k.time);

        match index {
            Ok(index) => Ok((&self.keyframes[index], index)),
            Err(index) => Err(index),
        }
    }
}

pub enum TaggedAnimation {
    Float(Animation<f32>),
}

impl TaggedAnimation {
    pub fn animate(&self, time: u32) -> TaggedValue {
        match self {
            Self::Float(animation) => animation.animate(time).into(),
        }
    }
}