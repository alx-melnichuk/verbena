import { Uri } from './uri';

// List of public methods that do not require authorization.
export type PublicMethods = { [key: string]: string };
const GET = 'GET';
const POST = 'POST';
const PUT = 'PUT';

export const LIST_PUBLIC_METHODS_PROFILE: PublicMethods = {
    [Uri.appUri('appApi://login')]: POST, // 'appApi://profile/auth'
    [Uri.appUri('appApi://token')]: POST, //   [Uri.appUri('appApi://profile/token')]: POST,
    //   [Uri.appUri('appApi://profile?userId=')]: GET,
    [Uri.appUri('appApi://registration')]: POST, // 'appApi://profile/registration'
    //   [Uri.appUri('appApi://profile/registration/')]: PUT, // 'profile/registration/:confirmationId'
    //   [Uri.appUri('appApi://profile/recovery')]: POST, // 'profile/recovery'
    //   [Uri.appUri('appApi://profile/recovery/')]: PUT, // 'profile/recovery/:confirmationId'
    //   [Uri.appUri('appApi://profile/stocks/')]: GET,  // 'profile/stocks/:userId', 'profile/stocks/search/:key'
};

export const LIST_PUBLIC_METHODS_STREAM: PublicMethods = {
    //   [Uri.appUri('appApi://streams/popular/tags')]: GET,
    //   [Uri.appUri('appApi://streams/calendar/')]: GET, // 'streams/calendar/:userId/:date'
    //   [Uri.appUri('appApi://streams')]: GET,
};

export const LIST_PUBLIC_METHODS: PublicMethods = {
    ...LIST_PUBLIC_METHODS_PROFILE,
    ...LIST_PUBLIC_METHODS_STREAM,
};
