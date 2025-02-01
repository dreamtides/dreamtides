#nullable enable

using System;
using Dreamcaller.Layout;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class Registry : MonoBehaviour
  {
    [SerializeField] Camera? _mainCamera;
    public Camera MainCamera => Check(_mainCamera);

    [SerializeField] ObjectLayout? _userHand;
    public ObjectLayout UserHand => Check(_userHand);

    [SerializeField] ObjectLayout? _enemyHand;
    public ObjectLayout EnemyHand => Check(_enemyHand);

    [SerializeField] ObjectLayout? _userDeck;
    public ObjectLayout UserDeck => Check(_userDeck);

    [SerializeField] ObjectLayout? _enemyDeck;
    public ObjectLayout EnemyDeck => Check(_enemyDeck);

    [SerializeField] ObjectLayout? _userBattlefield;
    public ObjectLayout UserBattlefield => Check(_userBattlefield);

    [SerializeField] ObjectLayout? _enemyBattlefield;
    public ObjectLayout EnemyBattlefield => Check(_enemyBattlefield);

    [SerializeField] ObjectLayout? _userVoid;
    public ObjectLayout UserVoid => Check(_userVoid);

    [SerializeField] ObjectLayout? _enemyVoid;
    public ObjectLayout EnemyVoid => Check(_enemyVoid);

    [SerializeField] ObjectLayout? _offscreen;
    public ObjectLayout Offscreen => Check(_offscreen);

    [SerializeField] ObjectLayout? _drawnCardsPosition;
    public ObjectLayout DrawnCardsPosition => Check(_drawnCardsPosition);

    [SerializeField] LayoutUpdateService? _layoutUpdateService;
    public LayoutUpdateService LayoutUpdateService => Check(_layoutUpdateService);

    [SerializeField] ActionService? _actionService;
    public ActionService ActionService => Check(_actionService);

    [SerializeField] InputService? _inputService;
    public InputService InputService => Check(_inputService);

    [SerializeField] DocumentService? _documentService;
    public DocumentService DocumentService => Check(_documentService);

    [SerializeField] CapabilitiesService? _capabilitiesService;
    public CapabilitiesService CapabilitiesService => Check(_capabilitiesService);

    [SerializeField] SoundService? _soundService;
    public SoundService SoundService => Check(_soundService);

    [SerializeField] CardService? _cardService;
    public CardService CardService => Check(_cardService);

    void Start()
    {
      Application.targetFrameRate = 60;

      foreach (var service in GetComponentsInChildren<Service>())
      {
        service.Initialize(this);
      }
    }

    T Check<T>(T? value) where T : UnityEngine.Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}