import { Routes } from '@angular/router';

import { P_STREAM_ID, E_STREAM_EDIT, E_STREAM_CREATE, E_STREAM_LIST } from '../common/routes';

import { PgStreamComponent } from './pg-stream.component';
import { pgStreamResolver } from './pg-stream.resolver';
import { pgProfileResolver } from './pg-profile.resolver';
import { pgStreamConfigResolver } from './pg-stream-config.resolver';
import { PgStreamListComponent } from './pg-stream-list/pg-stream-list.component';
import { PgStreamEditComponent } from './pg-stream-edit/pg-stream-edit.component';

export const PG_STREAM_ROUTES: Routes = [
    {
        path: '',
        component: PgStreamComponent,
        children: [
            {
                path: E_STREAM_LIST, // 'ind/stream/list'
                component: PgStreamListComponent,
                resolve: { profileDto: pgProfileResolver }
            },
            {
                path: E_STREAM_EDIT + '/:' + P_STREAM_ID, // 'ind/stream/edit/:streamId'
                component: PgStreamEditComponent,
                resolve: {
                    streamDto: pgStreamResolver,
                    streamConfigDto: pgStreamConfigResolver,
                }
            },
            {
                path: E_STREAM_CREATE, // 'ind/stream/create'
                component: PgStreamEditComponent,
                resolve: {
                    streamDto: pgStreamResolver,
                    streamConfigDto: pgStreamConfigResolver,
                }
            },
        ]
    },
];
