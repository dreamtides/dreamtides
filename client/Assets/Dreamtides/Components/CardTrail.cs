#nullable enable

using Dreamtides.Layout;
using UnityEngine;

namespace Dreamtides.Components
{
  [DisallowMultipleComponent]
  public sealed class CardTrail : MonoBehaviour
  {
    float _initializedAt;
    float? _durationSeconds;

    public void Initialize(float? durationSeconds = null)
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
      if (_durationSeconds != null && Time.time - _initializedAt > _durationSeconds.Value)
      {
        Destroy(gameObject);
      }
    }
  }
}
