import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';

@Component({
    selector: 'app-pg-concept',
    standalone: true,
    imports: [CommonModule, RouterOutlet],
    templateUrl: './pg-concept.component.html',
    styleUrl: './pg-concept.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgConceptComponent {
    constructor() {
    }
}
