import { DiscordSDK } from "@discord/embedded-app-sdk";
let discordSdk, auth, user
export function initializeDiscordSDK() {
    if (discordSdk) return
    else if (window.location.search.includes("instance_id")) {
        discordSdk = new DiscordSDK(window.location.hostname.split(".")[0]);
        setupSDK().then(r => console.log("sdk setup"))
    } else {
        discordSdk = null;
    }
    return discordSdk
}

async function setupSDK() {
    await discordSdk.ready();
    const { code } = await discordSdk.commands.authorize({
        client_id: window.location.hostname.split(".")[0],
        response_type: "code",
        state: "",
        prompt: "none",
        scope: [
            "identify"
        ],
    });

    const response = await fetch("/.proxy/api/token", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({code}),
    });

    const { access_token } = await response.json();
    auth = await discordSdk.commands.authenticate({ access_token });

    user = await fetch("https://discord.com/api/v10/users/@me", {
        headers: {
            Authorization: `Bearer ${auth.access_token}`,
            'Content-Type': 'application/json',
        },
    }).then((response) => user = response.json());
}

export function getGuildId() {
    return discordSdk ? discordSdk.guildId : "";
}

export function getChannelId() {
    return discordSdk ? discordSdk.channelId : "";
}

export function getUserId() {
    return discordSdk && user ? user.id : null;
}

export function getUserName() {
    return discordSdk ? user.username : null;
}

export function getDisplayName() {
    return discordSdk ? user.global_name : null;
}

export function getUserAvatar() {
    return discordSdk ? `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.webp` : null;
}

export function openExternalLink(url) {
    if (discordSdk) discordSdk.commands.openExternalLink({url});
}
