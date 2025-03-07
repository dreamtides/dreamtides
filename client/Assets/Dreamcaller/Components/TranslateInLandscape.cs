#nullable enable

using UnityEngine;

namespace Dreamcaller.Components
{
  public class TranslateInLandscape : MonoBehaviour
  {
    [SerializeField] Vector3 _translation = Vector3.zero;

    void Start()
    {
      transform.position += Screen.width >= Screen.height ? _translation : Vector3.zero;
    }
  }
}