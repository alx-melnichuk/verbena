import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { RouterOutlet } from '@angular/router';

@Component({
    selector: 'app-pg-banned',
    standalone: true,
    imports: [CommonModule, RouterOutlet],
    templateUrl: './pg-banned.component.html',
    styleUrl: './pg-banned.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgBannedComponent {
    constructor() {
    }
}
