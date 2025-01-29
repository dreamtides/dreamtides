#nullable enable

using System.Runtime.InteropServices;
using Dreamcaller.Schema;
using Dreamcaller.Utils;

static class Plugin
{
    const int BufferSize = 1_000_000;

    public static CommandSequence Connect()
    {
        byte[] response = new byte[BufferSize];
        int responseLength = Errors.CheckNonNegative(dreamcaller_connect(response, BufferSize));
        var json = System.Text.Encoding.UTF8.GetString(response, 0, responseLength);
        return CommandSequence.FromJson(json);
    }

    public static CommandSequence GetScene(int scene)
    {
        byte[] response = new byte[BufferSize];
        int responseLength = Errors.CheckNonNegative(dreamcaller_get_scene(scene, response, BufferSize));
        var json = System.Text.Encoding.UTF8.GetString(response, 0, responseLength);
        return CommandSequence.FromJson(json);
    }

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamcaller_connect([Out] byte[] response, int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamcaller_get_scene(int scene, [Out] byte[] response, int responseLength);
}
