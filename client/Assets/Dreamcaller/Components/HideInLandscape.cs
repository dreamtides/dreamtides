#nullable enable

using UnityEngine;

namespace Dreamcaller.Components
{
  public class HideInLanscape : MonoBehaviour
  {
    void Start()
    {
      gameObject.SetActive(Screen.width < Screen.height);
    }
  }
}