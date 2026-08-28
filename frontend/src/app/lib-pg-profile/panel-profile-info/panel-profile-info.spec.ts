import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfileInfo } from './panel-profile-info';

describe('PanelProfileInfo', () => {
  let component: PanelProfileInfo;
  let fixture: ComponentFixture<PanelProfileInfo>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelProfileInfo]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelProfileInfo);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
