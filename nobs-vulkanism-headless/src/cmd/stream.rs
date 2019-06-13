pub use super::CmdBuffer;

/// Allows to use [push](../stream/struct.Stream.html#method.push) on a [Stream](../stream/struct.Stream.html)
///
/// This trait is implemented for several vulkan commands in the [commands module](../commands/index.html).
/// Structs may define more complex rendering logic in this traits implementation.
pub trait StreamPush {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer;
}

/// Allows to use [push_mut](../stream/struct.Stream.html#method.push) on a [StreamMut](../stream/struct.StreamMut.html)
///
/// Same as [StreamPush](trait.StreamPush.html) but lets one modify the command while it is pushed.
pub trait StreamPushMut {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer;
}

/// Command Stream wrapper
///
/// Needs [push](trait.Stream.html#method.push) (already implemented by [CmdBuffer](../struct.CmdBuffer.html)).
/// Pushing a command into a stream will call [StreamPush::enqueue](trait.StreamPush.html#method.enqueue).
///
/// Also defines methods for pushing Options and lambdas
pub trait Stream: Sized {
  /// Pushes a command into a stream.
  ///
  /// Any struct implementing the [StreamPush](commands/trait.StreamPush.html) trait can be pushed into the stream.
  /// This can be used to build more complex commands from many primitive ones and be able to push them with one call.
  fn push<T: StreamPush>(self, o: &T) -> Self;

  /// Pushes a command contained in the option into a stream. NOP if the Option is None.
  ///
  /// Any struct implementing the [StreamPush](commands/trait.StreamPush.html) trait can be pushed into the stream.
  /// This can be used to build more complex commands from many primitive ones and be able to push them with one call.
  fn push_if<T: StreamPush>(self, o: &Option<T>) -> Self {
    match o {
      Some(ref c) => self.push(c),
      None => self,
    }
  }

  /// Pushes a lambda into a stream.
  ///
  /// Can be used to push complex command logic into stream.
  fn push_fn<F: Fn(Self) -> Self>(self, f: F) -> Self {
    f(self)
  }
}

/// Command Stream wrapper
///
/// Same as [Stream](trait.Stream.html) but for mutable commands.
pub trait StreamMut: Stream + Sized {
  /// Pushes a command into a stream.
  ///
  /// See [push](struct.Stream.html#method.push). Calls to a mutable version, so that the pushed command can be modified.
  fn push_mut<T: StreamPushMut>(self, o: &mut T) -> Self;

  /// Pushes a command into a stream.
  ///
  /// See [push_if](struct.Stream.html#method.push_if). Calls to a mutable version, so that the pushed command can be modified.
  fn push_if_mut<T: StreamPushMut>(self, o: &mut Option<T>) -> Self {
    match o {
      Some(ref mut c) => self.push_mut(c),
      None => self,
    }
  }

  /// Pushes a lambda into a stream.
  ///
  /// Can be used to push complex command logic into stream.
  fn push_fnmut<F: FnMut(Self) -> Self>(self, mut f: F) -> Self {
    f(self)
  }
}
