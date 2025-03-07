#nullable enable

using Dreamcaller.Services;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class GameEnvironment : MonoBehaviour
  {
    [SerializeField] Transform _portrait = null!;
    [SerializeField] Transform _landscape = null!;

    public void Activate(Registry registry)
    {
      _portrait.gameObject.SetActive(registry.IsPortrait);
      _landscape.gameObject.SetActive(!registry.IsPortrait);
    }
  }
}