import { ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, Output, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
    selector: 'app-sidebar',
    exportAs: 'appSidebar',
    standalone: true,
    imports: [CommonModule],
    templateUrl: './sidebar.component.html',
    styleUrl: './sidebar.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class SidebarComponent {
    @Input()
    @HostBinding('attr.is-above-parent')
    public isAboveParent: boolean = false;
    @Input()
    @HostBinding('attr.is-open')
    public isOpen: boolean = false;

    @Output()
    readonly clickByVeil: EventEmitter<boolean> = new EventEmitter();

    constructor() {
    }

    public doClickByVeil(): void {
        this.clickByVeil.emit(true);
    }
}
