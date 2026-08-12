import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgStreamEdit } from './pg-stream-edit';

describe('PgStreamEdit', () => {
  let component: PgStreamEdit;
  let fixture: ComponentFixture<PgStreamEdit>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgStreamEdit]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgStreamEdit);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
