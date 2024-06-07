import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

declare var APP_ABOUT: any;

@Component({
  selector: 'app-about',
  standalone: true,
  imports: [CommonModule, TranslateModule,],
  templateUrl: './about.component.html',
  styleUrls: ['./about.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class AboutComponent {
  public title: string = '';
  public label: string = '';
  public frontendItems: string[] = [];
  public backendItem01 = this.appAbout['backend01'] || '';
  public backendItem02: string[] = this.appAbout['backend02'] || [];
  public backendItem03: string[] = this.appAbout['backend03'] || [];
  public linkSwaggerUi = '/swagger-ui/';
  public linkRapidoc = '/rapidoc';
  public linkRedoc = '/redoc';

  public get appAbout(): any {
    return APP_ABOUT || {};
  }
  public set appAbout(value: any) {
  }

  constructor(private translate: TranslateService,) {
    const appName = this.translate.instant('app.name') || '';
    this.title = this.translate.instant('about.title', { appName: appName }) || '';
    this.label = this.translate.instant('about.label', { appName: appName }) || '';
    this.frontendItems = this.translate.instant('about.frontend_items') || [];
  }
  
  // ** Public API **
  
  // ** Private API **
    
}
