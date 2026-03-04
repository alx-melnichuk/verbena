import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfile } from './panel-profile';

describe('PanelProfile', () => {
  let component: PanelProfile;
  let fixture: ComponentFixture<PanelProfile>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelProfile]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelProfile);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
