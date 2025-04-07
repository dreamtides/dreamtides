#nullable enable

using UnityEngine;

namespace Dreamcaller.Components
{
  public class DestroyAfterSeconds : MonoBehaviour
  {
    [SerializeField] float _seconds = 1f;

    void Start()
    {
      Destroy(gameObject, _seconds);
    }
  }
}