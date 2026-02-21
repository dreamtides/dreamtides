#nullable enable

namespace Abu
{
    /// <summary>
    /// Interface for scene walkers that traverse a UI system and produce a tree of scene nodes.
    /// Each game implements one or more walkers for its specific UI systems.
    /// </summary>
    public interface ISceneWalker
    {
        /// <summary>
        /// Walk the scene and return a root node containing the subtree for this walker's
        /// UI system. Interactive nodes should be registered with the provided ref registry
        /// during the walk so they can be looked up for click/hover/drag dispatch.
        /// </summary>
        AbuSceneNode Walk(RefRegistry refRegistry);
    }
}
