#nullable enable

using Dreamcaller.Components;
using Dreamcaller.Layout;

namespace Dreamcaller.Services
{
  public class CapabilitiesService : Service
  {
    public bool CanDragCards() => true;

    /// <summary>
    /// Returns true if a card browser is currently open, e.g. to select a card
    /// or view the contents of the void.
    /// </summary>
    public bool AnyBrowserOpen() => false;

    /// <summary>
    /// Can the user currently info zoom a card that exists in the provided
    /// GameContext to display a large format version of the card.
    /// </summary>
    public bool CanInfoZoom(GameContext gameContext)
    {
      if (Registry.DocumentService.IsAnyPanelOpen())
      {
        return false;
      }

      switch (gameContext)
      {
        case GameContext.Browser:
        case GameContext.RewardBrowser:
        case GameContext.CardZoneBrowser:
        case GameContext.Hand:
          return true;
        case GameContext.Battlefield:
          return !AnyBrowserOpen();
        default:
          return false;
      }
    }
  }
}