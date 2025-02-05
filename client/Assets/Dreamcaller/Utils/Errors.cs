#nullable enable

using System;
using Object = UnityEngine.Object;

namespace Dreamcaller.Utils
{
  public static class Errors
  {
    public static T CheckNotNull<T>(T? value, string message = "") where T : class
    {
      switch (value)
      {
        case null:
        case Object c when !c:
          // UnityEngine.Object has weird null behavior
          throw new NullReferenceException($"Expected a non-null object of type {typeof(T).FullName}. {message}");
        default:
          return value;
      }
    }

    public static T CheckNotNull<T>(T? value) where T : struct
    {
      if (!value.HasValue)
      {
        throw new ArgumentException($"Expected a non-null value of type {typeof(T).FullName}");
      }

      return value.Value;
    }

    public static T CheckNotDefault<T>(T value) where T : Enum
    {
      if (Equals(value, default(T)))
      {
        throw new ArgumentException($"Expected enum value of type {typeof(T).FullName} to have a non-default value.");
      }

      return value;
    }

    public static int CheckNonNegative(float value) => CheckNonNegative((int)value);

    public static int CheckNonNegative(int value, string message = "")
    {
      if (value < 0)
      {
        throw new ArgumentException($"Expected value {value} to be >= 0. {message}");
      }

      return value;
    }

    public static Exception UnknownEnumValue<T>(T value) where T : Enum =>
      new ArgumentException($"Unknown '{typeof(T).Name}' value: '{value}'");

    public static void CheckArgument(bool expression, string message)
    {
      if (!expression)
      {
        throw new ArgumentException(message);
      }
    }
  }
}
