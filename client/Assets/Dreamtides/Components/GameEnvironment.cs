#nullable enable

using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Components
{
  public class GameEnvironment : MonoBehaviour
  {
    [SerializeField]
    Transform _portrait = null!;

    [SerializeField]
    Transform _landscape = null!;

    public void Activate(Registry registry)
    {
      _portrait.gameObject.SetActive(!registry.IsLandscape);
      _landscape.gameObject.SetActive(registry.IsLandscape);
    }
  }
}
