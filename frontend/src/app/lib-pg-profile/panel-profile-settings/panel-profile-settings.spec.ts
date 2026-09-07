import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfileSettings } from './panel-profile-settings';

describe('PanelProfileSettings', () => {
  let component: PanelProfileSettings;
  let fixture: ComponentFixture<PanelProfileSettings>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelProfileSettings]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelProfileSettings);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
