#nullable enable

using System.Collections.Generic;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public sealed class AssetPoolService : Service
  {
    readonly Dictionary<int, List<GameObject>> _pools = new();

    public T Create<T>(T prefab, Vector3 position, Transform? parent = null) where T : Component =>
      ComponentUtils.Get<T>(Create(prefab.gameObject, position, parent));

    public GameObject Create(GameObject prefab, Vector3 position, Transform? parent = null)
    {
      var instanceId = prefab.GetInstanceID();
      if (_pools.ContainsKey(instanceId))
      {
        var list = _pools[instanceId];
        foreach (var pooledObject in list)
        {
          if (!pooledObject.activeSelf)
          {
            pooledObject.transform.SetParent(parent, worldPositionStays: false);
            pooledObject.transform.position = position;
            pooledObject.SetActive(value: true);
            return pooledObject;
          }
        }
      }
      else
      {
        _pools[instanceId] = new List<GameObject>();
      }

      var result = Instantiate(prefab, parent, worldPositionStays: false);
      result.transform.position = position;
      _pools[instanceId].Add(result.gameObject);
      return result;
    }
  }
}