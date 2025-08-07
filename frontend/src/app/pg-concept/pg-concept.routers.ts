import { Routes } from '@angular/router';

import { E_CONCEPT_LIST, E_CONCEPT_VIEW, P_CONCEPT_ID } from '../common/routes';

import { pgChatMessagesResolver } from './pg-chat-messages.resolver';
import { PgConceptComponent } from './pg-concept.component';
import { pgConceptResolver } from './pg-concept.resolver';
import { PgConceptListComponent } from './pg-concept-list/pg-concept-list.component';
import { PgConceptViewComponent } from './pg-concept-view/pg-concept-view.component';
import { pgProfileResolver } from './pg-profile.resolver';
import { pgProfileTokensResolver } from './pg-profile-tokens.resolver';

export const PG_CONCEPT_ROUTES: Routes = [
    {
        path: '',
        component: PgConceptComponent,
        children: [
            {
                path: E_CONCEPT_LIST, // 'ind/concept/list'
                component: PgConceptListComponent,
            },
            {
                path: E_CONCEPT_VIEW + '/:' + P_CONCEPT_ID, // 'ind/concept/view/:streamId'
                component: PgConceptViewComponent,
                resolve: {
                    profileDto: pgProfileResolver,
                    profileTokensDto: pgProfileTokensResolver,
                    chatMsgList: pgChatMessagesResolver,
                    conceptResponse: pgConceptResolver,
                }
            },
        ]
    },
];
