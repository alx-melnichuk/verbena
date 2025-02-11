import { Routes } from '@angular/router';

import { E_CONCEPT_LIST, E_CONCEPT_VIEW, P_CONCEPT_ID } from '../common/routes';
import { ConceptListComponent } from '../lib-concept/concept-list/concept-list.component';
import { ConceptViewComponent } from '../lib-concept/concept-view/concept-view.component';

import { PgConceptComponent } from './pg-concept.component';
import { pgConceptResolver } from './pg-concept.resolver';

export const PG_CONCEPT_ROUTES: Routes = [
    {
        path: '',
        component: PgConceptComponent,
        children: [
            {
                path: E_CONCEPT_LIST, // 'ind/concept/list'
                component: ConceptListComponent,
            },
            {
                path: E_CONCEPT_VIEW + '/:' + P_CONCEPT_ID, // 'ind/concept/view/:streamId'
                component: ConceptViewComponent,
                resolve: { streamDto: pgConceptResolver }
            },
        ]
    },
];
