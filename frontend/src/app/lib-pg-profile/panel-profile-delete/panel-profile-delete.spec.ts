import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfileDelete } from './panel-profile-delete';

describe('PanelProfileDelete', () => {
  let component: PanelProfileDelete;
  let fixture: ComponentFixture<PanelProfileDelete>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelProfileDelete]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelProfileDelete);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
