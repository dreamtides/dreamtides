#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Services;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class GameEnvironment : MonoBehaviour
  {
    [SerializeField]
    internal Transform _portrait = null!;

    [SerializeField]
    internal Transform _landscape = null!;

    public void Activate(Registry registry)
    {
      _portrait.gameObject.SetActive(!registry.IsLandscape);
      _landscape.gameObject.SetActive(registry.IsLandscape);
    }
  }
}
