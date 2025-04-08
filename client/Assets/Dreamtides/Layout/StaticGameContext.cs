#nullable enable

using UnityEngine;

namespace Dreamtides.Layout
{
  /// <summary>
  /// A MonoBehaviour that sets the starting GameContext of its Displayable component
  /// </summary>
  public class StaticGameContext : MonoBehaviour
  {
    [SerializeField] GameContext _startingContext;

    void Start()
    {
      GetComponent<Displayable>().GameContext = _startingContext;
    }
  }
}
