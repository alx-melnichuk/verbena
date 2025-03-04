import { ChangeDetectionStrategy, Component, HostBinding, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '@ngx-translate/core';

declare var APP_ABOUT: any;

@Component({
    selector: 'app-panel-about',
    standalone: true,
    imports: [CommonModule, TranslatePipe],
    templateUrl: './panel-about.component.html',
    styleUrl: './panel-about.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelAboutComponent {
    public backendItem01 = this.appAbout['backend01'] || '';
    public backendItem02: string[] = this.appAbout['backend02'] || [];
    public backendItem03: string[] = this.appAbout['backend03'] || [];

    @HostBinding('class.global-scroll')
    public get isGlobalScroll(): boolean { return true; }

    public get appAbout(): any {
        return APP_ABOUT || {};
    }
    public set appAbout(value: any) {
    }

    constructor() {
    }

    // ** Public API **

    public getKey(item: string): string {
        const itemVal = (item || '');
        const n = itemVal.indexOf('=');
        return n > -1 ? itemVal.slice(0, n).trim() : '';
    }

    // ** Private API **

}
