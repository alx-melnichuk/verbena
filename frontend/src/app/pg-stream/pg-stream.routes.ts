import { Routes } from '@angular/router';

import { PgStreamComponent } from './pg-stream.component';

import { pgStreamResolver } from './pg-stream.resolver';

import { StreamEditComponent } from '../components/stream-edit/stream-edit.component';
import { P_STREAM_ID, E_STREAM_EDIT, E_STREAM_CREATE } from '../common/routes';

// pg-stream.routes
export const PG_STREAM_ROUTES: Routes = [
  {
    path: '',
    component: PgStreamComponent,
    children: [
    // { path: R_STREAM_LIST, component: PageStreamListComponent }, // 'list'
    {
      path: E_STREAM_EDIT + '/:' + P_STREAM_ID, // 'ind/stream/edit/:streamId'
      component: StreamEditComponent,
      resolve: { streamDto: pgStreamResolver }
    },
    // {
    //   path: R_STREAM_EDIT + '/:' + P_STREAM_ID,  // 'edit/:streamId'
    //   component: PageStreamComponent,
    //   resolve: { streamDTO: PageStreamResolver } ??
    // },
    {
      path: E_STREAM_CREATE, // 'ind/stream/create'
      component: StreamEditComponent,
      resolve: { streamDto: pgStreamResolver }
    },
    // {
    //   path: R_STREAM_CREATE,  // 'create'
    //   component: PageStreamComponent,
    //   resolve: { streamDTO: PageStreamResolver }
    // },
    ]
  },
];
