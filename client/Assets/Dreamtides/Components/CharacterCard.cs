#nullable enable
using Dreamtides.Layout;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Components
{
  public class CharacterCard : MonoBehaviour
  {
    [SerializeField] Renderer _characterImage = null!;
    [SerializeField] GameObject _testCharacterPrefab = null!;
    [SerializeField] Registry _registry = null!;

    void Start()
    {
      GetComponent<Card>().GameContext = GameContext.Hand;
      GetComponent<Card>()._registry = _registry;
      _registry.StudioService.CaptureSubject(_testCharacterPrefab, _characterImage, far: true);
    }
  }
}