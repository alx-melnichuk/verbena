import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, HostBinding, ViewEncapsulation } from '@angular/core';
import { environment as env } from "../../../environments/environment";
import { LOG_PG_ABOUT } from '../../common/routes';
import { PanelAboutInfo } from '../panel-about-info/panel-about-info';

declare var APP_ABOUT: any;

@Component({
    selector: 'app-pg-about-details',
    exportAs: "appPgAboutDetails",
    standalone: true,
    imports: [CommonModule, PanelAboutInfo],
    templateUrl: './pg-about-details.html',
    styleUrl: './pg-about-details.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgAboutDetails {

    public get appAbout(): any { return APP_ABOUT || {}; }
    public set appAbout(_val: any) { }

    public backendItem01 = this.appAbout["backend01"] || "";
    public backendItem02: string[] = this.appAbout["backend02"] || [];
    public backendItem03: string[] = this.appAbout["backend03"] || [];

    @HostBinding("class.global-scroll")
    public get isGlobalScroll(): boolean { return true; }

    constructor() {
        if (env.logLevel & LOG_PG_ABOUT) { console.info(`PgAboutDetails();`); }
    }
}
