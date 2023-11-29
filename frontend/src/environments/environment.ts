// This file can be replaced during build by using the `fileReplacements` array.
// `ng build` replaces `environment.ts` with `environment.prod.ts`.
// The list of file replacements can be found in `angular.json`.

declare var appEnv: any;

export const environment = {
  production: false,
  appRoot: (appEnv.appRoot || 'http://localhost:4250/'),
  // appApi: (appEnv.appApi || 'http://127.0.0.1:8080/'),
  appApi: (appEnv.appApi || 'https://127.0.0.1:8443/'),
};

/*
 * For easier debugging in development mode, you can import the following file
 * to ignore zone related error stack frames such as `zone.run`, `zoneDelegate.invokeTask`.
 *
 * This import should be commented out in production mode because it will have a negative impact
 * on performance if an error is thrown.
 */
// import 'zone.js/plugins/zone-error';  // Included with Angular CLI.
