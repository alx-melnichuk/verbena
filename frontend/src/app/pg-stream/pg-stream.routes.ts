import { Routes } from '@angular/router';

import { StreamEditComponent } from '../lib-stream/stream-edit/stream-edit.component';
import { StreamListComponent } from '../lib-stream/stream-list/stream-list.component';
import { P_STREAM_ID, E_STREAM_EDIT, E_STREAM_CREATE, E_STREAM_LIST } from '../common/routes';

import { PgStreamComponent } from './pg-stream.component';
import { pgStreamResolver } from './pg-stream.resolver';
import { pgProfileResolver } from './pg-profile.resolver';
import { pgStreamConfigResolver } from './pg-stream-config.resolver';

export const PG_STREAM_ROUTES: Routes = [
    {
        path: '',
        component: PgStreamComponent,
        children: [
            {
                path: E_STREAM_LIST, // 'ind/stream/list'
                component: StreamListComponent,
                resolve: { userDto: pgProfileResolver }
            },
            {
                path: E_STREAM_EDIT + '/:' + P_STREAM_ID, // 'ind/stream/edit/:streamId'
                component: StreamEditComponent,
                resolve: {
                    streamDto: pgStreamResolver,
                    streamConfigDto: pgStreamConfigResolver,
                }
            },
            {
                path: E_STREAM_CREATE, // 'ind/stream/create'
                component: StreamEditComponent,
                resolve: {
                    streamDto: pgStreamResolver,
                    streamConfigDto: pgStreamConfigResolver,
                }
            },
        ]
    },
];
