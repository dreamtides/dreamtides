#nullable enable

using Dreamcaller.Components;
using TMPro;
using UnityEngine;

namespace Dreamcaller
{
  public class BattlefieldNumber : MonoBehaviour
  {
    [SerializeField] TextMeshPro _text = null!;
    [SerializeField] TimedEffect _onChange = null!;

    public void SetNumber(long number, bool animate)
    {
      if (_text.text != number.ToString())
      {
        _text.text = number.ToString();
        if (animate)
        {
          // Toggle to restart animation if needed
          _onChange.gameObject.SetActive(false);
          _onChange.gameObject.SetActive(true);
        }
      }
    }
  }
}