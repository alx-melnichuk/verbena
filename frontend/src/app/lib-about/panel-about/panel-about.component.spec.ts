import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelAboutComponent } from './panel-about.component';

describe('PanelAboutComponent', () => {
  let component: PanelAboutComponent;
  let fixture: ComponentFixture<PanelAboutComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelAboutComponent]
    });
    fixture = TestBed.createComponent(PanelAboutComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
