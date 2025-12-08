#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  /// <summary>
  /// A MonoBehaviour that sets the starting GameContext of its Displayable component
  /// </summary>
  public class StaticGameContext : MonoBehaviour
  {
    [SerializeField]
    internal GameContext _startingContext;

    void Start()
    {
      GetComponent<Displayable>().GameContext = _startingContext;
    }
  }
}
