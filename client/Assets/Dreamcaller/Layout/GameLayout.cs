#nullable enable

namespace Dreamcaller.Layout
{
  using Dreamcaller.Components;
  using Dreamcaller.Utils;
  using UnityEngine;

  public class GameLayout : MonoBehaviour
  {
    [SerializeField] Camera? _mainCamera;
    public Camera MainCamera => Check(_mainCamera);

    [SerializeField] AudioSource? _mainAudioSource;
    public AudioSource MainAudioSource => Check(_mainAudioSource);

    [SerializeField] AudioSource? _musicAudioSource;
    public AudioSource MusicAudioSource => Check(_musicAudioSource);

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

    [SerializeField] ObjectLayout? _userAvatar;
    public ObjectLayout UserAvatar => Check(_userAvatar);

    [SerializeField] ObjectLayout? _enemyAvatar;
    public ObjectLayout EnemyAvatar => Check(_enemyAvatar);

    [SerializeField] ObjectLayout? _offscreen;
    public ObjectLayout Offscreen => Check(_offscreen);

    [SerializeField] ObjectLayout? _drawnCardsPosition;
    public ObjectLayout DrawnCardsPosition => Check(_drawnCardsPosition);

    [SerializeField] ObjectLayout? _stack;
    public ObjectLayout Stack => Check(_stack);

    [SerializeField] ObjectLayout? _selectingTargetsEnemy;
    public ObjectLayout SelectingTargetsEnemy => Check(_selectingTargetsEnemy);

    [SerializeField] ObjectLayout? _selectingTargetsUser;
    public ObjectLayout SelectingTargetsUser => Check(_selectingTargetsUser);

    [SerializeField] SpriteRenderer? _battlefieldOverlay;
    public SpriteRenderer BattlefieldOverlay => Check(_battlefieldOverlay);

    [SerializeField] GameMessage? _gameMessage;
    public GameMessage GameMessage => Check(_gameMessage);

    T Check<T>(T? value) where T : UnityEngine.Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}