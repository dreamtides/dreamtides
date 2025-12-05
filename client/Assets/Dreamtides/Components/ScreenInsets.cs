#nullable enable

using System;

namespace Dreamtides.Components
{
  [Serializable]
  public struct ScreenInsets : IEquatable<ScreenInsets>
  {
    public float Top;
    public float Left;
    public float Bottom;
    public float Right;

    public bool Equals(ScreenInsets other)
    {
      return Top.Equals(other.Top)
        && Left.Equals(other.Left)
        && Bottom.Equals(other.Bottom)
        && Right.Equals(other.Right);
    }

    public override bool Equals(object? obj)
    {
      return obj is ScreenInsets other && Equals(other);
    }

    public override int GetHashCode()
    {
      return HashCode.Combine(Top, Left, Bottom, Right);
    }
  }
}
