#nullable enable

using Dreamtides.Layout;
using UnityEngine;

namespace Dreamtides.Components
{
  [DisallowMultipleComponent]
  public sealed class CardTrail : MonoBehaviour
  {
    float _initializedAt;
    float _durationSeconds;

    public void Initialize(float durationSeconds)
    {
      _initializedAt = Time.time;
      _durationSeconds = durationSeconds;
      foreach (var ps in GetComponentsInChildren<ParticleSystem>())
      {
        var renderer = ps.GetComponent<ParticleSystemRenderer>();
        if (renderer != null)
        {
          renderer.sortingLayerID = GameContext.Battlefield.SortingLayerId();
        }
      }
    }

    void Update()
    {
      if (Time.time - _initializedAt > _durationSeconds)
      {
        Destroy(gameObject);
      }
    }
  }
}