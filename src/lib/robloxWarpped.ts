// please study this and add it into your booststraper, its really nice to have this in
import { invoke } from "@tauri-apps/api/core";

export interface Game {
    id: number;
}

export interface Session {
    id: number;
    game_id: number;
    started_at: number;
    ended_at: number | null;
    duration: number | null;
}

interface PlayerArchetype {
    name: string;
    description: string;
    associatedGameIds: number[];
}

export const PLAYER_CATEGORIES_2026: PlayerArchetype[] = [
    {
        name: "Ordinary person",
        description: "I dont have anything really much to say about you",
        associatedGameIds: [16732694052, 6516141723]
    },
    {
        name: "Battlegrounds players",
        description: "Either you are really good at the game or playing just for the trend/hype",
        associatedGameIds: [9391468976, 18519254033, 10449761463, 13076380114, 12891247010, 95886635953476, 113318245878384, 14266669489, 17015287443, 15269951959]
    },
    {
        name: "FPS players",
        description: "That mean you have really good aiming! Keep up",
        associatedGameIds: [12629753855, 3297964905, 15509312676, 5938036553, 286090429, 122446657157717, 17625359962, 109397169461300, 292439477, 136801880565837, 94590879393563, 12144402492]
    },
    {
        name: "Limbus Company Fan",
        description: "what?",
        associatedGameIds: [17122706530]
    },
    {
        name: "Parkour player",
        description: "You played a lot of parkour games! Thats cool",
        associatedGameIds: [107205390939183, 445664957, 16166423027, 131273255138534, 9800976141]
    },
    {
        name: "Big undertale fan",
        description: "You play a lot of undertale inspired games! Are you having a bad time?",
        associatedGameIds: [12283052444, 7491927311, 109378285407321, 14986545779, 3198259055]
    },
    {
        name: "Anime stuff?",
        description: "I dont know how to name this category",
        associatedGameIds: [18668065416, 130739873848552, 13621938427]
    },
    {
        name: "Rhymes games player",
        description: "I see it there! But dont waste your talent on Roblox. play osu!",
        associatedGameIds: [119886179319425, 7216838953, 106809703371658, 13042495892, 116543366018035, 6813414321, 18651095883]
    },
    {
        name: "Fucking genius",
        description: "i would never figure out how to play these games",
        associatedGameIds: [166986752, 9192423027, 9798463281, 82461912126439, 133238741253706, 537413528]
    },
    {
        name: '"I found treasure in places nobody would ever look!"',
        description: "You played so many hidden gems. I'm proud of you!",
        associatedGameIds: [7138009149, 93978595733734, 129279692364812, 97719631053849, 16483433878, 107467295209358, 9798463281]
    },
    {
        name: "Do you perhaps need help?",
        description: "i'd listen you out",
        associatedGameIds: [90307869155475, 102259817027922, 138548983302972, 135266079827175, 13628338181]
    },
    {
        name: "Obby creator",
        description: "I really like these games! I used to play alot",
        associatedGameIds: [2913303231, 8646026339]
    },
    {
        name: "Obby players",
        description: 'Either you are a kid who play obby all day or you are very skilled!',
        associatedGameIds: [8562822414, 3240225109, 15873244701, 119011107416066]
    },
    {
        name: "E-daters or just weird",
        description: "i'm sorry what",
        associatedGameIds: [4282985734, 10449761463, 7041939546, 9391468976]
    },
    {
        name: "Cringe/Corny",
        description: "im sorry.",
        associatedGameIds: [7041939546, 16991287194, 4972273297, 16116270224, 10834586502, 18687417158]
    },
    {
        name: "Gooner",
        description: "please get a job",
        associatedGameIds: [88519281604403, 79305036070450, 93794424681636, 109715918987082, 70654235200182, 89100823864844, 117426257317557, 127875646547457]
    },
    {
        name: "Brainrotted 💀",
        description: "bro like 5 years old",
        associatedGameIds: [126884695634066, 79546208627805, 103525960981185, 107646426076756, 101736964164901, 115921318088549, 16732694052, 75888315541325, 139403706409507, 105742951729183, 137069154816703, 109983668079237]
    }
];

export function getPlayedGames(): Promise<Game[]> {
    return invoke("get_played_games");
}

export function getPlayedSessions(): Promise<Session[]> {
    return invoke("get_played_sessions");
}

export function personallyDeterminePlayerType(history: Session[], categories: PlayerArchetype[]): string {
    const statsByGame = new Map<number, { totalDuration: number; sessionCount: number }>();
    for (const session of history) {
        const existing = statsByGame.get(session.game_id) ?? { totalDuration: 0, sessionCount: 0 };
        statsByGame.set(session.game_id, {
            totalDuration: existing.totalDuration + (session.duration ?? 0),
            sessionCount: existing.sessionCount + 1,
        });
    }

    let bestMatchName = "idk lol";
    let maxScore = 0;

    categories.forEach(category => {
        let categoryScore = 0;

        category.associatedGameIds.forEach(id => {
            const stats = statsByGame.get(id);
            if (!stats) return;
            const avgDuration = stats.totalDuration / stats.sessionCount;
            categoryScore += stats.sessionCount * avgDuration;
        });

        if (categoryScore > maxScore) {
            maxScore = categoryScore;
            bestMatchName = category.name;
        }
    });

    return bestMatchName;
}

export function getMostPlayedGame(sessions: Session[]) {
    const durationMap: Record<number, number> = {};

    sessions.forEach(session => {
        if (session.duration) {
            durationMap[session.game_id] = (durationMap[session.game_id] || 0) + session.duration;
        }
    });

    let mostPlayedGameId: string | null = null;
    let maxDuration = -1;

    Object.entries(durationMap).forEach(([gameId, duration]) => {
        if (duration > maxDuration) {
            maxDuration = duration;
            mostPlayedGameId = gameId;
        }
    });

    return mostPlayedGameId ? Number(mostPlayedGameId) : null;
}