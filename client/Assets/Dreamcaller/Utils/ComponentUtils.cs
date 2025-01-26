#nullable enable

using System;
using UnityEngine;

namespace Dreamcaller.Utils
{
  public static class ComponentUtils
  {
    public static T Instantiate<T>(T prefabComponent, Vector3 position) where T : Component =>
      InstantiateGameObject<T>(Errors.CheckNotNull(prefabComponent).gameObject, position);

    public static T InstantiateGameObject<T>(GameObject prefab, Vector3 position) where T : Component
    {
      Errors.CheckNotNull(prefab);
      var instantiated = UnityEngine.Object.Instantiate(prefab, position, Quaternion.identity);
      var result = instantiated.GetComponent<T>();
      if (!result)
      {
        throw new NullReferenceException($"Expected a component of type {typeof(T).FullName}");
      }
      return result;
    }
  }
}
