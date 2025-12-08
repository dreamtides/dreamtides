#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class DestroyAfterSeconds : MonoBehaviour
  {
    [SerializeField]
    internal float _seconds = 1f;

    void Start()
    {
      Destroy(gameObject, _seconds);
    }
  }
}
