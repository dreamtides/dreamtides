#nullable enable

using System;
using UnityEngine;

namespace Dreamtides.Utils
{
  public static class ComponentUtils
  {
    public static T Instantiate<T>(T prefabComponent, Vector3? position = null) where T : Component =>
      InstantiateGameObject<T>(Errors.CheckNotNull(prefabComponent).gameObject, position);

    public static T InstantiateGameObject<T>(GameObject prefab, Vector3? position = null) where T : Component
    {
      Errors.CheckNotNull(prefab);
      var instantiated = UnityEngine.Object.Instantiate(prefab, position ?? 1000f * Vector3.one, Quaternion.identity);
      var result = instantiated.GetComponent<T>();
      if (!result)
      {
        throw new NullReferenceException($"Expected a component of type {typeof(T).FullName}");
      }
      return result;
    }

    public static T Get<T>(Component component) where T : Component
    {
      Errors.CheckNotNull(component);
      var result = component.GetComponent<T>();
      if (!result)
      {
        throw new NullReferenceException(
          $"Expected a component of type {typeof(T).FullName} on {component.gameObject.name}");
      }

      return result;
    }

    public static T Get<T>(GameObject gameObject) where T : Component
    {
      Errors.CheckNotNull(gameObject);
      var result = gameObject.GetComponent<T>();
      if (!result)
      {
        throw new NullReferenceException($"Expected a component of type {typeof(T).FullName} on {gameObject.name}");
      }

      return result;
    }
  }
}
