import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';

import { ConceptListComponent } from 'src/app/lib-concept/concept-list/concept-list.component';

@Component({
    selector: 'app-pg-concept-list',
    standalone: true,
    imports: [CommonModule, ConceptListComponent],
    templateUrl: './pg-concept-list.component.html',
    styleUrl: './pg-concept-list.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgConceptListComponent {

}
