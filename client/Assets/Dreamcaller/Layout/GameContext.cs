#nullable enable

using UnityEngine;

namespace Dreamcaller.Layout
{
  /// <summary>
  /// Represents the current location of a GameObject.
  /// </summary>
  public enum GameContext
  {
    // Note: Enum numbers are serialized by Unity and cannot be changed.
    //
    // Keep in sync with the 'Sorting Layers' tab in Unity
    Unspecified = 0,
    Hidden = 1,
    BehindBattlefield = 2,
    Battlefield = 3,
    Dreamsigns = 4,
    Deck = 5,
    DiscardPile = 6,
    PlayerAvatar = 7,
    ArenaHighlight = 8,
    HandStorage = 9,
    Interface = 10,
    Stack = 11,
    Drawn = 12,
    RevealedCards = 13,
    Hand = 14,
    Browser = 15,
    BrowserDragTarget = 16,
    Effects = 17,
    Dragging = 18,
    UserMessage = 19,
    CardZoneBrowser = 20,
    RewardBrowser = 21,
    InfoZoom = 22,
    SplashScreen = 23,
  }

  public static class SortingLayerExtensions
  {
    public static int SortingLayerId(this GameContext context) => context switch
    {
      GameContext.Hidden => SortingLayer.NameToID("Hidden"),
      GameContext.BehindBattlefield => SortingLayer.NameToID("Behind Battlefield"),
      GameContext.Battlefield => SortingLayer.NameToID("Battlefield"),
      GameContext.Dreamsigns => SortingLayer.NameToID("Dreamsigns"),
      GameContext.Deck => SortingLayer.NameToID("Deck"),
      GameContext.DiscardPile => SortingLayer.NameToID("Discard Pile"),
      GameContext.PlayerAvatar => SortingLayer.NameToID("Player Avatar"),
      GameContext.ArenaHighlight => SortingLayer.NameToID("Arena Highlight"),
      GameContext.HandStorage => SortingLayer.NameToID("Hand Storage"),
      GameContext.Interface => SortingLayer.NameToID("Interface"),
      GameContext.Stack => SortingLayer.NameToID("Stack"),
      GameContext.Drawn => SortingLayer.NameToID("Drawn"),
      GameContext.RevealedCards => SortingLayer.NameToID("Revealed Cards"),
      GameContext.Hand => SortingLayer.NameToID("Hand"),
      GameContext.Browser => SortingLayer.NameToID("Browser"),
      GameContext.BrowserDragTarget => SortingLayer.NameToID("Browser Drag Target"),
      GameContext.Effects => SortingLayer.NameToID("Effects"),
      GameContext.Dragging => SortingLayer.NameToID("Dragging"),
      GameContext.UserMessage => SortingLayer.NameToID("User Message"),
      GameContext.CardZoneBrowser => SortingLayer.NameToID("Card Zone Browser"),
      GameContext.RewardBrowser => SortingLayer.NameToID("Reward Browser"),
      GameContext.InfoZoom => SortingLayer.NameToID("Info Zoom"),
      GameContext.SplashScreen => SortingLayer.NameToID("Splash Screen"),
      _ => 0
    };

    public static bool IsBattlefieldContext(this GameContext context) => context switch
    {
      GameContext.BehindBattlefield => true,
      GameContext.Battlefield => true,
      GameContext.Dreamsigns => true,
      GameContext.DiscardPile => true,
      _ => false
    };
  }
}